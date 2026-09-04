//! Bounded, identity-checked local configuration and static-asset intake.
//!
//! These readers adopt regular files through no-follow handles.  Host
//! acceptance parsing deliberately stops at its directly named identities;
//! it does not recursively parse scientific result or review payloads.

use crate::strict_json;
use crate::wire::{
    is_hex, ArtifactIdentity, AssetFile, AssetManifest, FileIdentity, HostAcceptance, HostIdentity,
    LocalConfiguration, NativeQualification, ARTIFACT_BYTES, ARTIFACT_SHA256, ASSET_SCHEMA,
    BIND_HOST, CONFIGURED_MODEL_ID, CONFIG_SCHEMA, FIRST_TARGET, HOST_ACCEPTANCE_SCHEMA,
    NATIVE_CONTRACT_SHA256, NATIVE_QUALIFICATION_SCHEMA, NATIVE_STATE_SHA256, OPERATOR_PROFILE,
    ORIGINAL_EXPORT_RELEASE_SHA256, RAW_INPUT_SCHEMA,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};

pub const LOCAL_CONFIGURATION_BYTES: u64 = 65_536;
pub const HOST_ACCEPTANCE_BYTES: u64 = 65_536;
pub const QUALIFICATION_BYTES: u64 = 65_536;
pub const RUNTIME_RECEIPT_BYTES: u64 = 1_048_576;
pub const FRESH_EXECUTION_RELEASE_BYTES: u64 = 1_048_576;
pub const ACCEPTED_RESULT_REVIEW_BYTES: u64 = 1_048_576;
pub const COMPARISON_RESULT_BYTES: u64 = 8_388_608;
pub const ASSET_MANIFEST_BYTES: u64 = 131_072;
pub const ASSET_ENTRIES: usize = 128;
pub const PER_ASSET_BYTES: u64 = 4_194_304;
pub const TOTAL_ASSET_BYTES: u64 = 16_777_216;
pub const EXECUTABLE_BYTES: u64 = 268_435_456;
pub const STREAM_HASH_BUFFER_BYTES: usize = 65_536;
pub const ABSOLUTE_PATH_UTF8_BYTES: usize = 4_096;
pub const RELATIVE_ASSET_PATH_UTF8_BYTES: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntakeErrorKind {
    InvalidPath,
    InvalidIdentity,
    TooLarge,
    NotRegular,
    Unavailable,
    InvalidJson,
    InvalidSchema,
    DuplicateAsset,
    MissingIndex,
    UnsupportedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntakeError {
    pub kind: IntakeErrorKind,
    message: String,
}

impl IntakeError {
    fn new(kind: IntakeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn io(error: io::Error) -> Self {
        let kind = match error.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => {
                IntakeErrorKind::Unavailable
            }
            _ => IntakeErrorKind::NotRegular,
        };
        Self::new(kind, "file could not be adopted through a no-follow handle")
    }
}

impl std::fmt::Display for IntakeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for IntakeError {}

#[derive(Debug, Clone)]
pub struct VerifiedFile<T> {
    pub path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
    pub raw_bytes: Vec<u8>,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAsset {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub mime: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedAssets {
    pub manifest: VerifiedFile<AssetManifest>,
    pub files: BTreeMap<String, VerifiedAsset>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct AdoptedHostAcceptance {
    pub acceptance: VerifiedFile<HostAcceptance>,
    pub qualification_bytes: Vec<u8>,
    pub qualification: NativeQualification,
    pub runtime_identity: uor_r4_api::learned_reference::RuntimeIdentity,
}

/// Configuration state safe for the parent to retain before any model load.
/// The artifact is represented only by its accepted identity; its bytes have
/// not been opened or read.
#[derive(Debug, Clone)]
pub struct ValidatedConfiguration {
    pub configuration: VerifiedFile<LocalConfiguration>,
    pub host: HostIdentity,
    pub configured_artifact: ArtifactIdentity,
    pub assets: VerifiedAssets,
    pub host_acceptance: Option<AdoptedHostAcceptance>,
    /// Why an optional host acceptance could not be adopted. Root
    /// configuration and asset failures are still returned from
    /// `load_configuration`; only this evidence lane degrades to discovery-only
    /// service availability.
    pub host_acceptance_error: Option<IntakeError>,
}

pub fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn validate_absolute_path(path: &Path) -> Result<(), IntakeError> {
    let text = path
        .to_str()
        .ok_or_else(|| IntakeError::new(IntakeErrorKind::InvalidPath, "path is not UTF-8"))?;
    if !path.is_absolute() || text.as_bytes().len() > ABSOLUTE_PATH_UTF8_BYTES {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidPath,
            "path is not an admitted absolute path",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn open_absolute_nofollow(path: &Path) -> Result<File, IntakeError> {
    validate_absolute_path(path)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    options.open(path).map_err(IntakeError::io)
}

#[cfg(not(unix))]
fn open_absolute_nofollow(_: &Path) -> Result<File, IntakeError> {
    Err(IntakeError::new(
        IntakeErrorKind::UnsupportedRuntime,
        "no-follow intake is unavailable on this target",
    ))
}

fn validate_regular_length(file: &File, declared: u64, maximum: u64) -> Result<(), IntakeError> {
    if declared > maximum {
        return Err(IntakeError::new(
            IntakeErrorKind::TooLarge,
            "declared file length exceeds its cap",
        ));
    }
    let metadata = file.metadata().map_err(IntakeError::io)?;
    if !metadata.file_type().is_file() {
        return Err(IntakeError::new(
            IntakeErrorKind::NotRegular,
            "adopted handle is not a regular file",
        ));
    }
    if metadata.len() != declared {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "declared and actual file lengths differ",
        ));
    }
    Ok(())
}

fn read_open_file(mut file: File, declared: u64, maximum: u64) -> Result<Vec<u8>, IntakeError> {
    validate_regular_length(&file, declared, maximum)?;
    let capacity = usize::try_from(declared).map_err(|_| {
        IntakeError::new(IntakeErrorKind::TooLarge, "file length is not addressable")
    })?;
    let read_cap = maximum
        .checked_add(1)
        .ok_or_else(|| IntakeError::new(IntakeErrorKind::TooLarge, "file cap overflow"))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.by_ref()
        .take(read_cap)
        .read_to_end(&mut bytes)
        .map_err(IntakeError::io)?;
    if bytes.len() as u64 != declared {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "file changed while being read",
        ));
    }
    Ok(bytes)
}

fn validate_identity_shape(identity: &FileIdentity, maximum: u64) -> Result<PathBuf, IntakeError> {
    let path = PathBuf::from(&identity.path);
    validate_absolute_path(&path)?;
    if identity.bytes > maximum || !is_hex(&identity.sha256, 64) {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "file identity is outside its accepted shape or cap",
        ));
    }
    Ok(path)
}

/// Read one absolute, regular, no-follow file and verify its exact identity.
pub fn read_file_identity(identity: &FileIdentity, maximum: u64) -> Result<Vec<u8>, IntakeError> {
    let path = validate_identity_shape(identity, maximum)?;
    let file = open_absolute_nofollow(&path)?;
    let bytes = read_open_file(file, identity.bytes, maximum)?;
    if sha256(&bytes) != identity.sha256 {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "file digest does not match adopted identity",
        ));
    }
    Ok(bytes)
}

/// Stream-hash a potentially large regular file with the frozen 64 KiB buffer.
pub fn hash_absolute_regular_file(path: &Path, maximum: u64) -> Result<(u64, String), IntakeError> {
    let mut file = open_absolute_nofollow(path)?;
    let metadata = file.metadata().map_err(IntakeError::io)?;
    if !metadata.file_type().is_file() {
        return Err(IntakeError::new(
            IntakeErrorKind::NotRegular,
            "adopted handle is not a regular file",
        ));
    }
    let length = metadata.len();
    if length > maximum {
        return Err(IntakeError::new(
            IntakeErrorKind::TooLarge,
            "file exceeds stream-hash cap",
        ));
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; STREAM_HASH_BUFFER_BYTES];
    let mut observed = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(IntakeError::io)?;
        if count == 0 {
            break;
        }
        observed = observed.checked_add(count as u64).ok_or_else(|| {
            IntakeError::new(IntakeErrorKind::TooLarge, "streamed byte count overflow")
        })?;
        if observed > maximum {
            return Err(IntakeError::new(
                IntakeErrorKind::TooLarge,
                "file grew beyond stream-hash cap",
            ));
        }
        digest.update(&buffer[..count]);
    }
    if observed != length {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "file changed while being hashed",
        ));
    }
    let digest = digest.finalize();
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    Ok((observed, output))
}

/// Check the API-owned binding and the independently supplied frozen JSON
/// binding without reinterpreting either value.
pub fn validate_expected_binding(
    binding: &uor_r4_api::learned_reference::ExpectedBinding,
    frozen_accepted_binding: &Value,
) -> Result<(), IntakeError> {
    let valid = binding.artifact_sha256 == ARTIFACT_SHA256
        && binding.contract_sha256 == NATIVE_CONTRACT_SHA256
        && binding.accepted_binding == *frozen_accepted_binding
        && binding.operator_profile == OPERATOR_PROFILE
        && binding.export_release_sha256 == ORIGINAL_EXPORT_RELEASE_SHA256;
    if !valid {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "expected binding differs from the frozen native contract",
        ));
    }
    Ok(())
}

pub fn validate_local_configuration(
    configuration: &LocalConfiguration,
    frozen_accepted_binding: &Value,
) -> Result<(), IntakeError> {
    if configuration.schema != CONFIG_SCHEMA
        || configuration.bind_host != BIND_HOST
        || !(1024..=65_535).contains(&configuration.port)
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "local configuration literals are invalid",
        ));
    }
    validate_absolute_path(Path::new(&configuration.artifact_path))?;
    validate_expected_binding(&configuration.expected_binding, frozen_accepted_binding)?;
    validate_identity_shape(&configuration.asset_manifest, ASSET_MANIFEST_BYTES)?;

    // Host-acceptance coherence belongs to the optional evidence lane below.
    // Missing, incomplete, malformed, or mismatched evidence leaves discovery
    // available with UNAVAILABLE_NATIVE_QUALIFICATION instead of rejecting the
    // root configuration and preventing the listener from starting.
    Ok(())
}

pub fn read_local_configuration(
    path: &Path,
    frozen_accepted_binding: &Value,
) -> Result<VerifiedFile<LocalConfiguration>, IntakeError> {
    validate_absolute_path(path)?;
    let file = open_absolute_nofollow(path)?;
    let declared = file.metadata().map_err(IntakeError::io)?.len();
    let bytes = read_open_file(file, declared, LOCAL_CONFIGURATION_BYTES)?;
    let value = strict_json::from_slice::<LocalConfiguration>(&bytes)
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidJson, error.to_string()))?;
    validate_local_configuration(&value, frozen_accepted_binding)?;
    Ok(VerifiedFile {
        path: path.to_owned(),
        bytes: declared,
        sha256: sha256(&bytes),
        raw_bytes: bytes,
        value,
    })
}

pub fn validate_host_acceptance(
    acceptance: &HostAcceptance,
    service_contract_sha256: &str,
) -> Result<(), IntakeError> {
    if acceptance.schema != HOST_ACCEPTANCE_SCHEMA
        || !is_hex(service_contract_sha256, 64)
        || acceptance.service_contract_sha256 != service_contract_sha256
        || !is_hex(&acceptance.native_binary_sha256, 64)
        || acceptance.operator_profile != OPERATOR_PROFILE
        || acceptance.target != FIRST_TARGET
        || acceptance.original_export_release_sha256 != ORIGINAL_EXPORT_RELEASE_SHA256
        || acceptance.artifact_sha256 != ARTIFACT_SHA256
        || acceptance.native_state_sha256 != NATIVE_STATE_SHA256
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "host acceptance bindings differ from the frozen service contract",
        ));
    }

    for (identity, cap) in [
        (&acceptance.runtime_receipt, RUNTIME_RECEIPT_BYTES),
        (&acceptance.qualification, QUALIFICATION_BYTES),
        (&acceptance.comparison_result, COMPARISON_RESULT_BYTES),
        (
            &acceptance.accepted_result_review,
            ACCEPTED_RESULT_REVIEW_BYTES,
        ),
        (
            &acceptance.fresh_execution_release,
            FRESH_EXECUTION_RELEASE_BYTES,
        ),
    ] {
        validate_identity_shape(identity, cap)?;
    }
    Ok(())
}

/// Parse only the adopted HostAcceptance object. Its nested result, review,
/// release, and runtime evidence identities remain opaque in this layer.
pub fn read_host_acceptance(
    identity: &FileIdentity,
    trusted_sha256: &str,
    service_contract_sha256: &str,
) -> Result<VerifiedFile<HostAcceptance>, IntakeError> {
    if identity.sha256 != trusted_sha256 || !is_hex(trusted_sha256, 64) {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "host acceptance does not match the operator-adopted digest",
        ));
    }
    let bytes = read_file_identity(identity, HOST_ACCEPTANCE_BYTES)?;
    let acceptance = strict_json::from_slice::<HostAcceptance>(&bytes)
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidJson, error.to_string()))?;
    validate_host_acceptance(&acceptance, service_contract_sha256)?;
    Ok(VerifiedFile {
        path: PathBuf::from(&identity.path),
        bytes: identity.bytes,
        sha256: identity.sha256.clone(),
        raw_bytes: bytes,
        value: acceptance,
    })
}

fn emit_ascii_json(value: &Value, output: &mut Vec<u8>) -> Result<(), IntakeError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(value) => output.extend_from_slice(if *value {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        }),
        Value::Number(number) => {
            let value = number.as_u64().ok_or_else(|| {
                IntakeError::new(
                    IntakeErrorKind::InvalidSchema,
                    "canonical authority JSON permits only unsigned integers",
                )
            })?;
            output.extend_from_slice(value.to_string().as_bytes());
        }
        Value::String(value) => {
            if !value.is_ascii() {
                return Err(IntakeError::new(
                    IntakeErrorKind::InvalidSchema,
                    "canonical authority JSON strings must be ASCII",
                ));
            }
            output.push(b'"');
            for byte in value.bytes() {
                match byte {
                    b'"' => output.extend_from_slice(b"\\\""),
                    b'\\' => output.extend_from_slice(b"\\\\"),
                    0..=31 => {
                        const HEX: &[u8; 16] = b"0123456789abcdef";
                        output.extend_from_slice(b"\\u00");
                        output.push(HEX[(byte >> 4) as usize]);
                        output.push(HEX[(byte & 0x0f) as usize]);
                    }
                    _ => output.push(byte),
                }
            }
            output.push(b'"');
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                emit_ascii_json(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                emit_ascii_json(&Value::String(key.clone()), output)?;
                output.push(b':');
                emit_ascii_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn parse_native_qualification(
    bytes: &[u8],
    acceptance: &HostAcceptance,
    actual_binary_sha256: &str,
) -> Result<NativeQualification, IntakeError> {
    let value = strict_json::from_slice::<Value>(bytes)
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidJson, error.to_string()))?;
    let mut canonical = Vec::with_capacity(bytes.len());
    emit_ascii_json(&value, &mut canonical)?;
    if canonical != bytes {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "qualification is not canonical ascii-json-1086/1",
        ));
    }
    let qualification = strict_json::from_slice::<NativeQualification>(bytes)
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidJson, error.to_string()))?;
    let valid = qualification.schema == NATIVE_QUALIFICATION_SCHEMA
        && qualification.terminal == "NATIVE_REFERENCE_PRESERVED"
        && qualification.accepted_review_sha256 == acceptance.accepted_result_review.sha256
        && qualification.comparison_result_sha256 == acceptance.comparison_result.sha256
        && qualification.artifact_sha256 == acceptance.artifact_sha256
        && qualification.native_state_sha256 == acceptance.native_state_sha256
        && qualification.native_binary_sha256 == actual_binary_sha256
        && qualification.runtime_receipt_sha256 == acceptance.runtime_receipt.sha256
        && qualification.contract_sha256 == NATIVE_CONTRACT_SHA256
        && qualification.operator_profile == OPERATOR_PROFILE
        && qualification.request_schema == RAW_INPUT_SCHEMA
        && qualification.result_schema == "uor-r4.text-binding-result/1";
    if !valid {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "qualification fields do not match the adopted host acceptance",
        ));
    }
    Ok(qualification)
}

fn binding_string<'a>(binding: &'a Value, path: &[&str]) -> Result<&'a str, IntakeError> {
    let mut value = binding;
    for key in path {
        value = value.get(*key).ok_or_else(|| {
            IntakeError::new(
                IntakeErrorKind::InvalidSchema,
                "frozen binding lacks an artifact identity field",
            )
        })?;
    }
    value.as_str().ok_or_else(|| {
        IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "frozen binding artifact identity field is not a string",
        )
    })
}

pub fn artifact_identity_from_binding(
    frozen_accepted_binding: &Value,
) -> Result<ArtifactIdentity, IntakeError> {
    let identity = ArtifactIdentity {
        model_id: CONFIGURED_MODEL_ID.to_owned(),
        artifact_sha256: ARTIFACT_SHA256.to_owned(),
        artifact_bytes: ARTIFACT_BYTES,
        native_state_sha256: NATIVE_STATE_SHA256.to_owned(),
        codec_cid: binding_string(frozen_accepted_binding, &["assets", "vocabulary", "cid"])?
            .to_owned(),
        policy_sha256: binding_string(frozen_accepted_binding, &["policy_sha256"])?.to_owned(),
        reader_file_cid: binding_string(frozen_accepted_binding, &["assets", "reader", "cid"])?
            .to_owned(),
        core_file_cid: binding_string(frozen_accepted_binding, &["assets", "core", "cid"])?
            .to_owned(),
        frame_tree_cid: binding_string(frozen_accepted_binding, &["frame_tree_cid"])?.to_owned(),
        original_export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
    };
    identity
        .validate()
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidSchema, error.to_string()))?;
    Ok(identity)
}

/// Validate the complete local parent intake without opening the model.
///
/// Opaque result/review/release/runtime records are read only far enough to
/// verify their adopted byte identities and then discarded. The canonical
/// qualification bytes are retained for the worker's later model-bound
/// qualification check.
pub fn load_configuration(
    path: &Path,
    expected_configuration_sha256: &str,
    actual_binary_sha256: &str,
    frozen_accepted_binding: &Value,
    service_contract_sha256: &str,
) -> Result<ValidatedConfiguration, IntakeError> {
    if !is_hex(expected_configuration_sha256, 64)
        || !is_hex(actual_binary_sha256, 64)
        || !is_hex(service_contract_sha256, 64)
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "startup digest is not lowercase hex64",
        ));
    }
    let configuration = read_local_configuration(path, frozen_accepted_binding)?;
    if configuration.sha256 != expected_configuration_sha256 {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "configuration does not match the independently supplied digest",
        ));
    }
    let assets = read_asset_manifest(&configuration.value.asset_manifest)?;
    let configured_artifact = artifact_identity_from_binding(frozen_accepted_binding)?;

    let adoption = (|| -> Result<Option<AdoptedHostAcceptance>, IntakeError> {
        match (
            configuration.value.host_acceptance.as_ref(),
            configuration
                .value
                .trusted_host_acceptance_sha256
                .as_deref(),
        ) {
            (None, None) => Err(IntakeError::new(
                IntakeErrorKind::Unavailable,
                "current host has no independently adopted qualification",
            )),
            (Some(identity), Some(trusted)) => {
                validate_identity_shape(identity, HOST_ACCEPTANCE_BYTES)?;
                if !is_hex(trusted, 64) || trusted != identity.sha256.as_str() {
                    return Err(IntakeError::new(
                        IntakeErrorKind::InvalidIdentity,
                        "host acceptance identity is not independently adopted",
                    ));
                }
                let acceptance = read_host_acceptance(identity, trusted, service_contract_sha256)?;
                if acceptance.value.native_binary_sha256 != actual_binary_sha256 {
                    return Err(IntakeError::new(
                        IntakeErrorKind::InvalidIdentity,
                        "host acceptance names different executable bytes",
                    ));
                }

                // These four payloads stay opaque. Reading verifies exact bytes,
                // bounded lengths, regular-file status, and adopted digests.
                drop(read_file_identity(
                    &acceptance.value.runtime_receipt,
                    RUNTIME_RECEIPT_BYTES,
                )?);
                let qualification_bytes =
                    read_file_identity(&acceptance.value.qualification, QUALIFICATION_BYTES)?;
                let qualification = parse_native_qualification(
                    &qualification_bytes,
                    &acceptance.value,
                    actual_binary_sha256,
                )?;
                drop(read_file_identity(
                    &acceptance.value.comparison_result,
                    COMPARISON_RESULT_BYTES,
                )?);
                drop(read_file_identity(
                    &acceptance.value.accepted_result_review,
                    ACCEPTED_RESULT_REVIEW_BYTES,
                )?);
                drop(read_file_identity(
                    &acceptance.value.fresh_execution_release,
                    FRESH_EXECUTION_RELEASE_BYTES,
                )?);

                Ok(Some(AdoptedHostAcceptance {
                    runtime_identity: uor_r4_api::learned_reference::RuntimeIdentity {
                        native_binary_sha256: actual_binary_sha256.to_owned(),
                        runtime_receipt_sha256: acceptance.value.runtime_receipt.sha256.clone(),
                    },
                    qualification_bytes,
                    qualification,
                    acceptance,
                }))
            }
            _ => Err(IntakeError::new(
                IntakeErrorKind::InvalidSchema,
                "host acceptance configuration is incoherent",
            )),
        }
    })();
    let (adopted, host_acceptance_error) = match adoption {
        Ok(adopted) => (adopted, None),
        Err(error) => (None, Some(error)),
    };

    let host = HostIdentity {
        native_binary_sha256: actual_binary_sha256.to_owned(),
        runtime_receipt_sha256: adopted
            .as_ref()
            .map(|value| value.acceptance.value.runtime_receipt.sha256.clone()),
        target: FIRST_TARGET.to_owned(),
        operator_profile: OPERATOR_PROFILE.to_owned(),
        service_contract_sha256: service_contract_sha256.to_owned(),
        asset_manifest_sha256: configuration.value.asset_manifest.sha256.clone(),
        host_acceptance_sha256: adopted
            .as_ref()
            .map(|value| value.acceptance.sha256.clone()),
        qualification_receipt_sha256: adopted
            .as_ref()
            .map(|value| value.acceptance.value.qualification.sha256.clone()),
    };
    host.validate()
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidSchema, error.to_string()))?;

    Ok(ValidatedConfiguration {
        configuration,
        host,
        configured_artifact,
        assets,
        host_acceptance: adopted,
        host_acceptance_error,
    })
}

/// Worker/private-mode boundary: open the configured model only after the
/// caller has completed all parent-side admission and selected model work.
pub fn read_exact_artifact(configuration: &ValidatedConfiguration) -> Result<Vec<u8>, IntakeError> {
    let identity = FileIdentity {
        path: configuration.configuration.value.artifact_path.clone(),
        bytes: ARTIFACT_BYTES,
        sha256: ARTIFACT_SHA256.to_owned(),
    };
    read_file_identity(&identity, ARTIFACT_BYTES)
}

pub fn validate_relative_asset_path(path: &str) -> Result<(), IntakeError> {
    if path.is_empty()
        || path.as_bytes().len() > RELATIVE_ASSET_PATH_UTF8_BYTES
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('%')
        || path.as_bytes().contains(&0)
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidPath,
            "asset path is not an admitted relative POSIX path",
        ));
    }
    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidPath,
            "asset path contains an empty, dot, or dotdot segment",
        ));
    }

    let lower = path.to_ascii_lowercase();
    let forbidden_suffixes = [
        ".bin",
        ".c",
        ".cc",
        ".cpp",
        ".h",
        ".ini",
        ".json",
        ".jsonl",
        ".lock",
        ".md",
        ".npy",
        ".npz",
        ".onnx",
        ".pt",
        ".pth",
        ".py",
        ".rs",
        ".safetensors",
        ".sh",
        ".toml",
        ".ts",
        ".tsx",
        ".yaml",
        ".yml",
    ];
    if forbidden_suffixes
        .iter()
        .any(|suffix| lower.ends_with(suffix))
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidPath,
            "source, configuration, model, and evidence files cannot be served as assets",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn c_path(bytes: &[u8]) -> Result<CString, IntakeError> {
    CString::new(bytes)
        .map_err(|_| IntakeError::new(IntakeErrorKind::InvalidPath, "path contains a null byte"))
}

#[cfg(unix)]
fn open_relative_nofollow(directory: &Path, relative: &str) -> Result<File, IntakeError> {
    validate_absolute_path(directory)?;
    validate_relative_asset_path(relative)?;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    let mut directory_file = options.open(directory).map_err(IntakeError::io)?;
    let mut components = relative.split('/').peekable();
    while let Some(component) = components.next() {
        let name = c_path(component.as_bytes())?;
        let directory_component = components.peek().is_some();
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | libc::O_NONBLOCK
            | if directory_component {
                libc::O_DIRECTORY
            } else {
                0
            };
        // SAFETY: `directory_file` owns a live descriptor, `name` is a
        // NUL-terminated single path component, and no create flag is used.
        let descriptor = unsafe { libc::openat(directory_file.as_raw_fd(), name.as_ptr(), flags) };
        if descriptor < 0 {
            return Err(IntakeError::io(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a fresh owned descriptor.
        let opened = unsafe { File::from_raw_fd(descriptor) };
        if directory_component {
            directory_file = opened;
        } else {
            return Ok(opened);
        }
    }
    Err(IntakeError::new(
        IntakeErrorKind::InvalidPath,
        "asset path has no file component",
    ))
}

#[cfg(not(unix))]
fn open_relative_nofollow(_: &Path, _: &str) -> Result<File, IntakeError> {
    Err(IntakeError::new(
        IntakeErrorKind::UnsupportedRuntime,
        "no-follow asset intake is unavailable on this target",
    ))
}

fn validate_asset_entry(entry: &AssetFile) -> Result<(), IntakeError> {
    validate_relative_asset_path(&entry.path)?;
    if entry.bytes == 0 || entry.bytes > PER_ASSET_BYTES || !is_hex(&entry.sha256, 64) {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidIdentity,
            "asset identity is empty, oversized, or malformed",
        ));
    }
    if entry.mime.is_empty()
        || entry.mime.len() > 256
        || !entry
            .mime
            .bytes()
            .all(|byte| byte == b' ' || (0x21..=0x7e).contains(&byte))
    {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "asset MIME is not a bounded printable-ASCII HTTP field value",
        ));
    }
    Ok(())
}

/// Adopt the exact manifest and every listed asset without following any
/// directory or file symlink. No unlisted directory member is read.
pub fn read_asset_manifest(identity: &FileIdentity) -> Result<VerifiedAssets, IntakeError> {
    let manifest_bytes = read_file_identity(identity, ASSET_MANIFEST_BYTES)?;
    let manifest = strict_json::from_slice::<AssetManifest>(&manifest_bytes)
        .map_err(|error| IntakeError::new(IntakeErrorKind::InvalidJson, error.to_string()))?;
    if manifest.schema != ASSET_SCHEMA || manifest.files.len() > ASSET_ENTRIES {
        return Err(IntakeError::new(
            IntakeErrorKind::InvalidSchema,
            "asset manifest schema or entry count is invalid",
        ));
    }

    let manifest_path = PathBuf::from(&identity.path);
    let directory = manifest_path.parent().ok_or_else(|| {
        IntakeError::new(
            IntakeErrorKind::InvalidPath,
            "asset manifest has no directory",
        )
    })?;
    let mut names = BTreeSet::new();
    let mut total = 0_u64;
    for entry in &manifest.files {
        validate_asset_entry(entry)?;
        if !names.insert(entry.path.clone()) {
            return Err(IntakeError::new(
                IntakeErrorKind::DuplicateAsset,
                "asset manifest contains a duplicate path",
            ));
        }
        total = total.checked_add(entry.bytes).ok_or_else(|| {
            IntakeError::new(IntakeErrorKind::TooLarge, "asset byte total overflow")
        })?;
        if total > TOTAL_ASSET_BYTES {
            return Err(IntakeError::new(
                IntakeErrorKind::TooLarge,
                "asset byte total exceeds its cap",
            ));
        }
    }
    if !names.contains("index.html") {
        return Err(IntakeError::new(
            IntakeErrorKind::MissingIndex,
            "asset manifest does not contain index.html",
        ));
    }

    let mut files = BTreeMap::new();
    for entry in &manifest.files {
        let file = open_relative_nofollow(directory, &entry.path)?;
        let bytes = read_open_file(file, entry.bytes, PER_ASSET_BYTES)?;
        if sha256(&bytes) != entry.sha256 {
            return Err(IntakeError::new(
                IntakeErrorKind::InvalidIdentity,
                "asset digest differs from manifest",
            ));
        }
        files.insert(
            entry.path.clone(),
            VerifiedAsset {
                bytes,
                sha256: entry.sha256.clone(),
                mime: entry.mime.clone(),
            },
        );
    }

    Ok(VerifiedAssets {
        manifest: VerifiedFile {
            path: manifest_path,
            bytes: identity.bytes,
            sha256: identity.sha256.clone(),
            raw_bytes: manifest_bytes,
            value: manifest,
        },
        files,
        total_bytes: total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "uor-r4-workbench-intake-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_identity(path: &Path, bytes: &[u8]) -> FileIdentity {
        fs::write(path, bytes).unwrap();
        FileIdentity {
            path: path.to_str().unwrap().to_owned(),
            bytes: bytes.len() as u64,
            sha256: sha256(bytes),
        }
    }

    fn manifest_identity(directory: &Path, files: Vec<AssetFile>) -> FileIdentity {
        let body = serde_json::to_vec(&AssetManifest {
            schema: ASSET_SCHEMA.to_owned(),
            files,
        })
        .unwrap();
        write_identity(&directory.join("assets.json"), &body)
    }

    fn synthetic_binding() -> Value {
        serde_json::json!({
            "assets": {
                "vocabulary": {"cid": format!("blake3:{}", "1".repeat(64))},
                "reader": {"cid": format!("blake3:{}", "2".repeat(64))},
                "core": {"cid": format!("blake3:{}", "3".repeat(64))}
            },
            "policy_sha256": "4".repeat(64),
            "frame_tree_cid": format!("blake3:{}", "5".repeat(64))
        })
    }

    fn synthetic_file(directory: &Path, name: &str, digit: char) -> FileIdentity {
        FileIdentity {
            path: directory.join(name).to_str().unwrap().to_owned(),
            bytes: 1,
            sha256: digit.to_string().repeat(64),
        }
    }

    fn synthetic_acceptance(directory: &Path) -> HostAcceptance {
        HostAcceptance {
            schema: HOST_ACCEPTANCE_SCHEMA.to_owned(),
            service_contract_sha256: "9".repeat(64),
            native_binary_sha256: "e".repeat(64),
            operator_profile: OPERATOR_PROFILE.to_owned(),
            target: FIRST_TARGET.to_owned(),
            runtime_receipt: synthetic_file(directory, "runtime.json", 'a'),
            qualification: synthetic_file(directory, "qualification.json", '6'),
            comparison_result: synthetic_file(directory, "comparison.json", 'b'),
            accepted_result_review: synthetic_file(directory, "review.json", 'c'),
            fresh_execution_release: synthetic_file(directory, "release.json", 'd'),
            original_export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
            artifact_sha256: ARTIFACT_SHA256.to_owned(),
            native_state_sha256: NATIVE_STATE_SHA256.to_owned(),
        }
    }

    #[test]
    fn absolute_identity_is_bounded_regular_and_exact() {
        let directory = TestDirectory::new();
        let identity = write_identity(&directory.0.join("record.json"), b"{}\n");
        assert_eq!(read_file_identity(&identity, 3).unwrap(), b"{}\n");

        let mut relative = identity.clone();
        relative.path = "record.json".to_owned();
        assert_eq!(
            read_file_identity(&relative, 3).unwrap_err().kind,
            IntakeErrorKind::InvalidPath
        );

        let mut wrong_length = identity.clone();
        wrong_length.bytes = 2;
        assert_eq!(
            read_file_identity(&wrong_length, 3).unwrap_err().kind,
            IntakeErrorKind::InvalidIdentity
        );
        assert_eq!(
            read_file_identity(&identity, 2).unwrap_err().kind,
            IntakeErrorKind::InvalidIdentity
        );
    }

    #[cfg(unix)]
    #[test]
    fn final_file_and_asset_directory_symlinks_are_not_followed() {
        use std::os::unix::fs::symlink;

        let directory = TestDirectory::new();
        let target = directory.0.join("target");
        fs::write(&target, b"bytes").unwrap();
        let link = directory.0.join("link");
        symlink(&target, &link).unwrap();
        let identity = FileIdentity {
            path: link.to_str().unwrap().to_owned(),
            bytes: 5,
            sha256: sha256(b"bytes"),
        };
        assert!(read_file_identity(&identity, 5).is_err());

        let real = directory.0.join("real");
        fs::create_dir(&real).unwrap();
        fs::write(real.join("index.html"), b"index").unwrap();
        symlink(&real, directory.0.join("linked-assets")).unwrap();
        assert!(open_relative_nofollow(&directory.0.join("linked-assets"), "index.html").is_err());
    }

    #[test]
    fn asset_paths_reject_traversal_escapes_and_sensitive_roles() {
        for path in [
            "",
            "/index.html",
            "../index.html",
            "a/../index.html",
            "a//index.html",
            "a\\index.html",
            "a%2findex.html",
            "src/main.rs",
            "model.safetensors",
            "evidence.jsonl",
        ] {
            assert!(validate_relative_asset_path(path).is_err(), "{path:?}");
        }
        for path in ["index.html", "app.js", "styles/main.css", "NOTICE.txt"] {
            validate_relative_asset_path(path).unwrap();
        }
    }

    #[test]
    fn asset_manifest_rejects_duplicates_and_declared_limits_before_reads() {
        let directory = TestDirectory::new();
        let duplicate = AssetFile {
            path: "index.html".to_owned(),
            bytes: 1,
            sha256: "0".repeat(64),
            mime: "text/html; charset=utf-8".to_owned(),
        };
        let identity = manifest_identity(&directory.0, vec![duplicate.clone(), duplicate]);
        assert_eq!(
            read_asset_manifest(&identity).unwrap_err().kind,
            IntakeErrorKind::DuplicateAsset
        );

        let oversized = AssetFile {
            path: "index.html".to_owned(),
            bytes: PER_ASSET_BYTES + 1,
            sha256: "0".repeat(64),
            mime: "text/html; charset=utf-8".to_owned(),
        };
        let identity = manifest_identity(&directory.0, vec![oversized]);
        assert_eq!(
            read_asset_manifest(&identity).unwrap_err().kind,
            IntakeErrorKind::InvalidIdentity
        );

        let too_many = (0..=ASSET_ENTRIES)
            .map(|index| AssetFile {
                path: if index == 0 {
                    "index.html".to_owned()
                } else {
                    format!("asset-{index}.js")
                },
                bytes: 1,
                sha256: "0".repeat(64),
                mime: "application/javascript; charset=utf-8".to_owned(),
            })
            .collect();
        let identity = manifest_identity(&directory.0, too_many);
        assert_eq!(
            read_asset_manifest(&identity).unwrap_err().kind,
            IntakeErrorKind::InvalidSchema
        );

        let excessive_total = (0..5)
            .map(|index| AssetFile {
                path: if index == 0 {
                    "index.html".to_owned()
                } else {
                    format!("asset-{index}.js")
                },
                bytes: PER_ASSET_BYTES,
                sha256: "0".repeat(64),
                mime: "application/javascript; charset=utf-8".to_owned(),
            })
            .collect();
        let identity = manifest_identity(&directory.0, excessive_total);
        assert_eq!(
            read_asset_manifest(&identity).unwrap_err().kind,
            IntakeErrorKind::TooLarge
        );

        let unicode_mime = AssetFile {
            path: "index.html".to_owned(),
            bytes: 1,
            sha256: "0".repeat(64),
            mime: "text/html; name=é".to_owned(),
        };
        assert_eq!(
            validate_asset_entry(&unicode_mime).unwrap_err().kind,
            IntakeErrorKind::InvalidSchema
        );
    }

    #[test]
    fn asset_manifest_adopts_only_exact_listed_regular_files() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("index.html"), b"<main></main>").unwrap();
        fs::write(directory.0.join("app.js"), b"'use strict';").unwrap();
        fs::write(directory.0.join("unlisted.txt"), b"not read").unwrap();
        let files = [
            (
                "index.html",
                b"<main></main>".as_slice(),
                "text/html; charset=utf-8",
            ),
            (
                "app.js",
                b"'use strict';".as_slice(),
                "application/javascript; charset=utf-8",
            ),
        ]
        .into_iter()
        .map(|(path, bytes, mime)| AssetFile {
            path: path.to_owned(),
            bytes: bytes.len() as u64,
            sha256: sha256(bytes),
            mime: mime.to_owned(),
        })
        .collect();
        let identity = manifest_identity(&directory.0, files);
        let assets = read_asset_manifest(&identity).unwrap();
        assert_eq!(assets.files.len(), 2);
        assert_eq!(assets.files["index.html"].bytes, b"<main></main>");
        assert!(!assets.files.contains_key("unlisted.txt"));
    }

    #[test]
    fn config_and_acceptance_structs_reject_extra_fields() {
        let directory = TestDirectory::new();
        let binding = synthetic_binding();
        let configuration = LocalConfiguration {
            schema: CONFIG_SCHEMA.to_owned(),
            artifact_path: directory.0.join("model.bin").to_str().unwrap().to_owned(),
            expected_binding: uor_r4_api::learned_reference::ExpectedBinding {
                artifact_sha256: ARTIFACT_SHA256.to_owned(),
                contract_sha256: NATIVE_CONTRACT_SHA256.to_owned(),
                accepted_binding: binding.clone(),
                operator_profile: OPERATOR_PROFILE.to_owned(),
                export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
            },
            host_acceptance: None,
            trusted_host_acceptance_sha256: None,
            asset_manifest: synthetic_file(&directory.0, "assets.json", '8'),
            bind_host: BIND_HOST.to_owned(),
            port: 8080,
        };
        validate_local_configuration(&configuration, &binding).unwrap();
        let mut value = serde_json::to_value(configuration).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("extra".to_owned(), Value::Null);
        assert!(strict_json::from_slice::<LocalConfiguration>(
            &serde_json::to_vec(&value).unwrap()
        )
        .is_err());

        let mut value = serde_json::to_value(synthetic_acceptance(&directory.0)).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("extra".to_owned(), Value::Null);
        assert!(
            strict_json::from_slice::<HostAcceptance>(&serde_json::to_vec(&value).unwrap())
                .is_err()
        );
    }

    #[test]
    fn canonical_qualification_requires_all_twelve_cross_bound_fields() {
        let directory = TestDirectory::new();
        let acceptance = synthetic_acceptance(&directory.0);
        let qualification = NativeQualification {
            schema: NATIVE_QUALIFICATION_SCHEMA.to_owned(),
            terminal: "NATIVE_REFERENCE_PRESERVED".to_owned(),
            accepted_review_sha256: acceptance.accepted_result_review.sha256.clone(),
            comparison_result_sha256: acceptance.comparison_result.sha256.clone(),
            artifact_sha256: acceptance.artifact_sha256.clone(),
            native_state_sha256: acceptance.native_state_sha256.clone(),
            native_binary_sha256: acceptance.native_binary_sha256.clone(),
            runtime_receipt_sha256: acceptance.runtime_receipt.sha256.clone(),
            contract_sha256: NATIVE_CONTRACT_SHA256.to_owned(),
            operator_profile: OPERATOR_PROFILE.to_owned(),
            request_schema: RAW_INPUT_SCHEMA.to_owned(),
            result_schema: "uor-r4.text-binding-result/1".to_owned(),
        };
        let value = serde_json::to_value(&qualification).unwrap();
        let mut canonical = Vec::new();
        emit_ascii_json(&value, &mut canonical).unwrap();
        assert_eq!(
            parse_native_qualification(&canonical, &acceptance, &acceptance.native_binary_sha256)
                .unwrap(),
            qualification
        );

        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        assert!(parse_native_qualification(
            &noncanonical,
            &acceptance,
            &acceptance.native_binary_sha256
        )
        .is_err());

        let mut mismatched = qualification;
        mismatched.comparison_result_sha256 = "f".repeat(64);
        let value = serde_json::to_value(mismatched).unwrap();
        let mut canonical = Vec::new();
        emit_ascii_json(&value, &mut canonical).unwrap();
        assert!(parse_native_qualification(
            &canonical,
            &acceptance,
            &acceptance.native_binary_sha256
        )
        .is_err());
    }

    #[test]
    fn parent_configuration_validation_does_not_open_the_model() {
        let directory = TestDirectory::new();
        let index = b"<main></main>";
        fs::write(directory.0.join("index.html"), index).unwrap();
        let asset_manifest = manifest_identity(
            &directory.0,
            vec![AssetFile {
                path: "index.html".to_owned(),
                bytes: index.len() as u64,
                sha256: sha256(index),
                mime: "text/html; charset=utf-8".to_owned(),
            }],
        );
        let binding = synthetic_binding();
        let configuration = LocalConfiguration {
            schema: CONFIG_SCHEMA.to_owned(),
            // Deliberately absent: parent admission must not open this file.
            artifact_path: directory
                .0
                .join("absent-model.bin")
                .to_str()
                .unwrap()
                .to_owned(),
            expected_binding: uor_r4_api::learned_reference::ExpectedBinding {
                artifact_sha256: ARTIFACT_SHA256.to_owned(),
                contract_sha256: NATIVE_CONTRACT_SHA256.to_owned(),
                accepted_binding: binding.clone(),
                operator_profile: OPERATOR_PROFILE.to_owned(),
                export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
            },
            host_acceptance: None,
            trusted_host_acceptance_sha256: None,
            asset_manifest,
            bind_host: BIND_HOST.to_owned(),
            port: 8080,
        };
        let configuration_bytes = serde_json::to_vec(&configuration).unwrap();
        let path = directory.0.join("workbench-config.json");
        fs::write(&path, &configuration_bytes).unwrap();
        let validated = load_configuration(
            &path,
            &sha256(&configuration_bytes),
            &"a".repeat(64),
            &binding,
            &"b".repeat(64),
        )
        .unwrap();
        assert!(validated.host_acceptance.is_none());
        assert_eq!(validated.assets.files.len(), 1);
        assert_eq!(
            validated.configured_artifact.artifact_sha256,
            ARTIFACT_SHA256
        );
    }
}
